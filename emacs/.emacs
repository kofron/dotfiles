(global-linum-mode 't)
(setq linum-format "%4d-\u2502 ")

(require 'ido)
(ido-mode t)

(require 'package)
(add-to-list 'package-archives
	     '("melpa" . "http://melpa.org/packages/") t)
(add-to-list 'package-archives
	     '("melpa" . "http://melpa.milkbox.net/packages/") t)
(when (< emacs-major-version 24)
  ;; For important compatibility libraries like cl-lib
  (add-to-list 'package-archives '("gnu" . "http://elpa.gnu.org/packages/")))
(package-initialize)

(when (not package-archive-contents) (package-refresh-contents))

;; R and ESS
(add-to-list 'load-path "~/.emacs.d/elpa/ess-20150616.357/lisp")
(load "ess-site")

;; scala/ensime
(require 'ensime)
(add-hook 'scala-mode-hook 'ensime-scala-mode-hook)

;; company mode foreva
(add-hook 'after-init-hook 'global-company-mode)
(global-set-key (kbd "C-x <up>") 'company-complete)

;; python company mode
(defun my/python-mode-hook ()
  (add-to-list 'company-backends 'company-jedi))

(add-hook 'python-mode-hook 'my/python-mode-hook)
