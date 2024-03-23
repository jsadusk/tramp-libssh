use std::cell::RefCell;
use std::io::{Seek, SeekFrom, Read};
use std::rc::Rc;
use emacs::{defun, CallEnv, Env, IntoLisp, Result, Value, FromLisp};

emacs::plugin_is_GPL_compatible!();

use libssh_rs::{*, sys::{SSH_ADDRSTRLEN, sftp_init}};
use anyhow::bail;
use libc::O_RDONLY;

/*thread_local! {
    static SESSIONS: RefCell<HashMap<String, Session>>;
}*/
emacs::use_symbols! {
    nil t
    car cdr nth
    tramp_dissect_file_name
    read_passwd read_string
    insert set_buffer current_buffer generate_new_buffer kill_buffer
}

#[emacs::module(name = "tramp-libssh")]
fn init(_: &Env) -> Result<()> { Ok(()) }

struct DissectedFilename {
    protocol: String,
    user: String,
    host: String,
    filename: String,
}

trait LocalEnv<'a> {
    fn car(&self, list: Value<'a>) -> Result<Value<'a>>;
    fn cdr(&self, list: Value<'a>) -> Result<Value<'a>>;
    fn nth(&self, idx: usize, list: Value<'a>) -> Result<Value<'a>>;
    fn tramp_dissect_file_name_el(&self, filename: Value<'a>) -> Result<Value<'a>>;
    fn tramp_dissect_file_name(&self, filename: Value<'a>) -> Result<DissectedFilename>;
    fn read_passwd(&self, prompt: &str, confirm: bool) -> Result<String>;
    fn read_string(&self, prompt: &str) -> Result<String>;
    fn insert(&self, text: &str) -> Result<()>;
    fn set_buffer(&self, buffer: Value<'a>) -> Result<()>;
    fn current_buffer(&self) -> Result<Value<'a>>;    
    fn generate_new_buffer(&self, name: &str) -> Result<Value<'a>>;
    fn kill_buffer(&self, buffer: Value<'a>) -> Result<()>;
}

impl<'a> LocalEnv<'a> for &'a Env {
    fn car(&self, list: Value<'a>) -> Result<Value<'a>> {
        self.call(car, &[list])
    }

    fn cdr(&self, list: Value<'a>) -> Result<Value<'a>> {
        self.call(cdr, &[list])
    }
    
    fn nth(&self, idx: usize, list: Value<'a>) -> Result<Value<'a>> {
        self.call(nth, &[idx.into_lisp(self)?, list])
    }

    fn tramp_dissect_file_name_el(&self, filename: Value<'a>) -> Result<Value<'a>> {
        self.call(tramp_dissect_file_name, &[filename])
    }

    fn tramp_dissect_file_name(&self, filename: Value<'a>) -> Result<DissectedFilename> {
        self.message("dissecting")?;
        let dissected_v = self.tramp_dissect_file_name_el(filename)?;
        self.message("extracting")?;
        
        Ok(
            DissectedFilename {
                protocol: String::from_lisp(self.nth(1, dissected_v)?)?,
                user: String::from_lisp(self.nth(2, dissected_v)?)?,
                host: String::from_lisp(self.nth(4, dissected_v)?)?,
                filename: String::from_lisp(self.nth(6, dissected_v)?)?,
            }
        )
    }

    fn read_passwd(&self, prompt: &str, confirm: bool) -> Result<String> {
        let confirm = if confirm {
            t
        } else {
            nil
        };
        let passwd_v = self.call(read_passwd, &[prompt.into_lisp(self)?, confirm.bind(self)])?;
        String::from_lisp(passwd_v)
    }

    fn read_string(&self, prompt: &str) -> Result<String> {
        let result_v = self.call(read_string, &[prompt.into_lisp(self)?])?;
        String::from_lisp(result_v)
    }

    fn insert(&self, text: &str) -> Result<()> {
        self.call(insert, &[text.into_lisp(self)?])?;
        Ok(())
    }

    fn set_buffer(&self, buffer: Value<'a>) -> Result<()> {
        self.call(set_buffer, &[buffer])?;
        Ok(())
    }

    fn current_buffer(&self) -> Result<Value<'a>> {
        self.call(current_buffer, &[])
    }

    fn generate_new_buffer(&self, name: &str) -> Result<Value<'a>> {
        self.call(generate_new_buffer, &[name.into_lisp(self)?])
    }

    fn kill_buffer(&self, buffer: Value<'a>) -> Result<()> {
        self.call(kill_buffer, &[buffer])?;
        Ok(())
    }
}

fn get_connection(user: &str, host: &str, env: &Env) -> Result<Session> {
    let session = Session::new()?;
    /*unsafe {
        //let env: *const LocalEnv = env as *const LocalEnv;
        session.set_auth_callback(
            |prompt, echo, verify, identity|
            env.borrow().read_passwd(prompt, verify).map_err(|e| libssh_rs::Error::Fatal(e.to_string()))
        );
    }*/
    session.set_option(SshOption::User(Some(user.to_string())))?;
    session.set_option(SshOption::Hostname(host.to_string()))?;
    session.options_parse_config(None)?;
    session.connect()?;
    let srv_pubkey = session.get_server_public_key()?;
    let hash = srv_pubkey.get_public_key_hash(PublicKeyHashType::Sha1)?;
    match session.is_known_server()? {
        KnownHosts::Changed => {
            bail!(format!("Host key for server {} changed", host));
        },
        KnownHosts::Other => {
            bail!(format!("Host key for server {} not found but other type of key exists", host));
        },
        KnownHosts::NotFound => {
            bail!("Known hosts file not found");
            
        },
        KnownHosts::Unknown => {
            bail!(format!("Server {} unknown", host));
        },
        KnownHosts::Ok => {}
    }

    session.userauth_public_key_auto(None, None)?;

    Ok(session)
}

#[defun]
fn insert_file_contents1(env: &Env, filename: Value, visit: Option<Value>, begin: Option<usize>, end: Option<usize>, replace: Option<Value>) -> Result<()> {
    env.message("insert-file-contents3")?;
    let dissected = env.tramp_dissect_file_name(filename)?;
    env.message("formatting")?;
    env.message(&format!("filename {} has username {} host {} and file {}", String::from_lisp(filename)?, dissected.user, dissected.host, dissected.filename))?;

    let session = get_connection(&dissected.user, &dissected.host, &env)?;
    env.message("Connected session")?;

    let sftp_sess = session.sftp()?;
    let mut rfile = sftp_sess.open(&dissected.filename, O_RDONLY, 0)?;
    if let Some(off) = begin {
        rfile.seek(SeekFrom::Start(off as u64))?;
    }

    let mut total_bytes: usize = 0;
    let mut buf = [0; 16384];
    loop {
        let bufslice: &mut [u8] = if let Some(end) = end {
            if total_bytes >= end {
                break;
            }

            let remaining: usize = end - total_bytes;
            if remaining < 16384 {
                &mut buf[0 .. remaining]
            } else {
                &mut buf[..]
            }
        } else {
            &mut buf[..]
        };
        
        let bytes = rfile.read(bufslice)?;
        if bytes == 0 {
            break;
        }
        
        env.insert(&std::str::from_utf8(&buf[0 .. bytes])?)?;

        total_bytes += bytes;
    }
        
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}

