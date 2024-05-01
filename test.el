;; This buffer is for text that is not saved, and for Lisp evaluation.
;; To create a file, visit it with C-x C-f and enter text in its buffer.

;; (add-to-list 'load-path "~/work/tramp-libssh/build")
;; (require 'emacs-libssh)

;; (defun test-emacs-libssh-get-session ()
;;   (interactive)
;;   (message "get session")
;;   (setq libssh-session (emacs-libssh-get-ssh-session nil "dev"))
;;   (message "get sftp")
;;   (setq libssh-sftp (emacs-libssh-get-sftp-session libssh-session))
;;   )


;; (defun test-emacs-libssh-insert ()
;;   (interactive)
;;   (message "sftp insert")
;;   (emacs-libssh-sftp-insert libssh-session libssh-sftp "/home/jsadusk/.bashrc" -1 -1)
;;   )

;; (defun test-emacs-libssh-insert-region ()
;;   (interactive)
;;   (message "sftp insert 20 - 500")
;;   (emacs-libssh-sftp-insert libssh-session libssh-sftp "/home/jsadusk/.bashrc" 23 590)
;;   )

;; (defun test-emacs-libssh-write-region ()
;;   (interactive)

;;   (message "sftp write 20 - 500")
;;   (emacs-libssh-sftp-write-region libssh-session libssh-sftp "/home/jsadusk/test_libssh.txt" 20 500 0)
;;   )


;; (let ((dissected (tramp-dissect-file-name "/ssh:joe@sadusk.com:/home/joe/hello.txt")))
;;   (message (prin1-to-string dissected))
;;   )

(add-to-list 'load-path "~/work/tramp-libssh")

(setq testhost "jsadusk@dev")
(setq testdir "/home/jsadusk/")
(setq testpath (concat "/ssh:" testhost ":" testdir))
(setq testfilename "missing.yaml")
(setq testfilepath (concat testpath testfilename))
(message testfilepath)
(require 'rs-module)
(defun test-libssh-insert-from-file ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-dissect-file-name "/ssh:joe@sadusk.com:/home/joe/data.txt")))
  (tramp-libssh-insert-file-contents1 "/ssh:joe@sadusk.com:/home/joe/data.txt" nil 4 15 nil)
  )
(defun test-libssh-replace-from-file ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-dissect-file-name "/ssh:joe@sadusk.com:/home/joe/data.txt")))
  (tramp-libssh-insert-file-contents1 "/ssh:joe@sadusk.com:/home/joe/data.txt" nil 4 15 t)
  )

(defun test-libssh-write-buffer ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (tramp-libssh-write-region nil nil testfilepath nil nil nil nil)
  )

(defun test-libssh-write-buffer-append ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (tramp-libssh-write-region nil nil testfilepath t nil nil nil)
  )

(defun test-libssh-file-exists ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-file-exists-p testfilepath)))
  (message (prin1-to-string (tramp-libssh-file-exists-p "/ssh:joe@sadusk.com:/home/joe/blarh.txt")))
  )

(defun test-directory-files ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-directory-files "/ssh:joe@sadusk.com:/home/joe/" nil nil nil nil)))
  )

(defun test-directory-files-fulldir ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-directory-files "/ssh:joe@sadusk.com:/home/joe/" t nil nil nil)))
  )

(defun test-directory-files-rexexp ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-directory-files "/ssh:joe@sadusk.com:/home/joe/" nil "^s.*t" nil nil)))
  )

(defun test-directory-files-count ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-directory-files "/ssh:joe@sadusk.com:/home/joe/" nil nil nil 5)))
  )

(defun test-delete-file ()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (tramp-libssh-delete-file testfilepath nil)
  )

(defun test-file-attributes()
  (interactive)
  (rs-module/load "work/tramp-libssh/target/debug/libtramp_libssh.dylib")
  (message (prin1-to-string (tramp-libssh-file-attributes testfilepath 'string)))
  )

;(message (read-string "hello: " nil nil nil nil))\
;(message (read-passwd "asdf: " 't))
