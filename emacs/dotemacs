(global-linum-mode 't)
(setq linum-format "%4d-\u2502 ")

(require 'ido)
(ido-mode t)

(require 'package)
(add-to-list 'package-archives
	     '("melpa" . "http://melpa.org/packages/") t)
;;(add-to-list 'package-archives
;;	     '("melpa" . "http://melpa.milkbox.net/packages/") t)
(when (< emacs-major-version 24)
  ;; For important compatibility libraries like cl-lib
  (add-to-list 'package-archives '("gnu" . "http://elpa.gnu.org/packages/")))
(package-initialize)

(when (not package-archive-contents) (package-refresh-contents))

;; R and ESS
;;(add-to-list 'load-path "~/.emacs.d/elpa/ess-20150616.357/lisp")
;;(load "ess-site")

;; scala/ensime
;;(require 'ensime)
;;(add-hook 'scala-mode-hook 'ensime-scala-mode-hook)

;; company mode foreva
(add-hook 'after-init-hook 'global-company-mode)
(global-set-key (kbd "C-x <up>") 'company-complete)

;; python company mode
(defun my/python-mode-hook ()
  (add-to-list 'company-backends 'company-jedi))

(add-hook 'python-mode-hook 'my/python-mode-hook)

;; rust
(load-file "~/dotfiles/emacs/rust/rust-init.el")
(custom-set-variables
 ;; custom-set-variables was added by Custom.
 ;; If you edit it by hand, you could mess it up, so be careful.
 ;; Your init file should contain only one such instance.
 ;; If there is more than one, they won't work right.
 '(custom-safe-themes
   (quote
    ("1e67765ecb4e53df20a96fb708a8601f6d7c8f02edb09d16c838e465ebe7f51b" "0ee3fc6d2e0fc8715ff59aed2432510d98f7e76fe81d183a0eb96789f4d897ca" default)))
 '(package-selected-packages
   (quote
    (magit yaml-mode rustfmt racer protobuf-mode paganini-theme material-theme markdown-mode flycheck-rust ess elixir-mode creamsody-theme company-racer company-go))))
(custom-set-faces
 ;; custom-set-faces was added by Custom.
 ;; If you edit it by hand, you could mess it up, so be careful.
 ;; Your init file should contain only one such instance.
 ;; If there is more than one, they won't work right.
 )
(setq custom-safe-themes t)
(load-theme 'material t)
