;;; $DOOMDIR/config.el -*- lexical-binding: t; -*-

;; Place your private configuration here! Remember, you do not need to run 'doom
;; sync' after modifying this file!


;; Some functionality uses this to identify you, e.g. GPG configuration, email
;; clients, file templates and snippets. It is optional.
;; (setq user-full-name "John Doe"
;;       user-mail-address "john@doe.com")

;; Doom exposes five (optional) variables for controlling fonts in Doom:
;;
;; - `doom-font' -- the primary font to use
;; - `doom-variable-pitch-font' -- a non-monospace font (where applicable)
;; - `doom-big-font' -- used for `doom-big-font-mode'; use this for
;;   presentations or streaming.
;; - `doom-symbol-font' -- for symbols
;; - `doom-serif-font' -- for the `fixed-pitch-serif' face
;;
;; See 'C-h v doom-font' for documentation and more examples of what they
;; accept. For example:
;;
(setq doom-font (font-spec :family "Iosevka" :size 10 :weight 'extralight)
      doom-variable-pitch-font (font-spec :family "Iosevka" :size 10))
;;
;; If you or Emacs can't find your font, use 'M-x describe-font' to look them
;; up, `M-x eval-region' to execute elisp code, and 'M-x doom/reload-font' to
;; refresh your font settings. If Emacs still can't find your font, it likely
;; wasn't installed correctly. Font issues are rarely Doom issues!

;; There are two ways to load a theme. Both assume the theme is installed and
;; available. You can either set `doom-theme' or manually load a theme with the
;; `load-theme' function. This is the default:
(setq doom-theme 'doom-moonlight)

;; This determines the style of line numbers in effect. If set to `nil', line
;; numbers are disabled. For relative line numbers, set this to `relative'.
(setq display-line-numbers-type t)

;; If you use `org' and don't want your org files in the default location below,
;; change `org-directory'. It must be set before org loads!
;;; --- org basics
(setq org-directory "~/org"
      org-agenda-files (list (expand-file-name "journal/" org-directory))
      org-log-done 'time
      org-todo-keywords '((sequence "TODO(t)" "FOCUS(f)" "MTG(m)" "WAIT(w)" "|" "DONE(d)" "KILL(x)"))
      org-tag-alist '((:startgroup)
                      ("work" . ?w) ("home" . ?h) ("dev" . ?d) ("idea" . ?i)
                      (:endgroup)
                      ("note" . ?N) ("reading" . ?r) ("deep" . ?D)))

;;; --- org-journal: daily files + carry-over
(defun +journal/open-today ()
  (interactive)
  (require 'org-journal)
  (org-journal-open-current-journal-file))

(defun +journal/new-entry ()
  (interactive)
  (require 'org-journal)
  (org-journal-new-entry t))

(defun +journal/search ()
  (interactive)
  (require 'org-journal)
  (org-journal-search))

(use-package! org-journal
  :after org
  :custom
  (org-journal-dir (expand-file-name "journal/" org-directory))
  (org-journal-file-type 'daily)
  (org-journal-file-format "%Y-%m-%d.org")
  (org-journal-date-format "%A, %d %B %Y")
  (org-journal-date-prefix "#+title: ")
  (org-journal-enable-agenda-integration t)
  ;; carry over unfinished items only
  (org-journal-carryover-items "TODO=\"TODO\"|TODO=\"NEXT\"|TODO=\"WAIT\"")
  ;; timestamped headings for quick notes
  (org-journal-time-prefix "* ")
  (org-journal-time-format "%H:%M "))

;;; --- capture: TODO (with time) + note into today's journal
(after! org
  (defun +journal-find-today ()
    "Ensure today’s journal exists and capture at end."
    (org-journal-new-entry t)
    (goto-char (point-max)))

  (add-to-list 'org-capture-templates '("j" "Journal"))

  (dolist (face '(("MTG" . (:foreground "#51afef" :weight bold))
                  ("FOCUS" . (:foreground "#da8548" :weight bold))))
    (setf (alist-get (car face) org-todo-keyword-faces nil nil #'string=)
          (cdr face)))

  (add-to-list 'org-capture-templates
               '("j t" "Journal TODO (today)" entry
                 (function +journal-find-today)
                 "* TODO %^{Task}\nSCHEDULED: %^{When|<%<%Y-%m-%d %H:%M>>}\n:PROPERTIES:\n:CREATED: %U\n:END:\n%?\n"))

  (add-to-list 'org-capture-templates
               '("j n" "Journal note (today)" entry
                 (function +journal-find-today)
                 "* %^{Title|Note} %^g\n:PROPERTIES:\n:CREATED: %U\n:END:\n%?\n")))



(with-eval-after-load 'org (global-org-modern-mode))
(package-initialize)
(menu-bar-mode -1)
(tool-bar-mode -1)
;; (scroll-bar-mode -1)
;; (modus-themes-load-operandi)

;; Choose some fonts
;; (set-face-attribute 'default nil :family "Iosevka")
;; (set-face-attribute 'variable-pitch nil :family "Iosevka Aile")
;; (set-face-attribute 'org-modern-symbol nil :family "Iosevka")

;; Add frame borders and window dividers
(modify-all-frames-parameters
 '((right-divider-width . 40)
   (internal-border-width . 40)))
(dolist (face '(window-divider
                window-divider-first-pixel
                window-divider-last-pixel))
  (face-spec-reset-face face)
  (set-face-foreground face (face-attribute 'default :background)))
(set-face-background 'fringe (face-attribute 'default :background))

(setq
 ;; Edit settings
 org-auto-align-tags nil
 org-tags-column 0
 org-catch-invisible-edits 'show-and-error
 org-special-ctrl-a/e t
 org-insert-heading-respect-content t

 ;; Org styling, hide markup etc.
 org-hide-emphasis-markers t
 org-pretty-entities t
 org-agenda-tags-column 0
 org-ellipsis "…")

(global-org-modern-mode)

;;; --- non-evil keybindings (plain Emacs)
;; journal
(global-set-key (kbd "C-c j t") #'+journal/open-today)             ; open/create today
(global-set-key (kbd "C-c j j") #'+journal/new-entry)              ; new entry in today
(global-set-key (kbd "C-c j s") #'+journal/search)

;; quick captures into today
(global-set-key (kbd "C-c j T") (lambda () (interactive) (org-capture nil "j t"))) ; TODO+time
(global-set-key (kbd "C-c j N") (lambda () (interactive) (org-capture nil "j n"))) ; note+tags

;; agenda
(global-set-key (kbd "C-c a") #'org-agenda)

;;; --- custom agenda focused on this wortkflow
(after! org-agenda
  (add-to-list 'org-agenda-custom-commands
               '("J" "Today + carry-over"
                 ((agenda "" ((org-agenda-span 1)
                              (org-agenda-start-day "0")
                              (org-agenda-show-all-dates t)))
                  (todo "NEXT")
                  (todo "TODO"))))
  (setq org-agenda-start-on-weekday nil))


;;; Journal template insertion
(defvar-local jk/journal-template-inserted nil
  "Track whether the journal day template has been applied in this buffer.")

(defun jk/insert-journal-template ()
  (let ((template-file (expand-file-name "template.journal.org" org-directory)))
    (when (and (not jk/journal-template-inserted)
               (file-readable-p template-file))
      (save-excursion
        (goto-char (point-max))
        (unless (bolp)
          (insert "\n"))
        (insert-file-contents template-file))
      (setq jk/journal-template-inserted t))))

(after! org-journal
  (add-hook 'org-journal-after-header-create-hook #'jk/insert-journal-template))


;; Whenever you reconfigure a package, make sure to wrap your config in an
;; `after!' block, otherwise Doom's defaults may override your settings. E.g.
;;
;;   (after! PACKAGE
;;     (setq x y))
;;
;; The exceptions to this rule:
;;
;;   - Setting file/directory variables (like `org-directory')
;;   - Setting variables which explicitly tell you to set them before their
;;     package is loaded (see 'C-h v VARIABLE' to look up their documentation).
;;   - Setting doom variables (which start with 'doom-' or '+').
;;
;; Here are some additional functions/macros that will help you configure Doom.
;;
;; - `load!' for loading external *.el files relative to this one
;; - `use-package!' for configuring packages
;; - `after!' for running code after a package has loaded
;; - `add-load-path!' for adding directories to the `load-path', relative to
;;   this file. Emacs searches the `load-path' when you load packages with
;;   `require' or `use-package'.
;; - `map!' for binding new keys
;;
;; To get information about any of these functions/macros, move the cursor over
;; the highlighted symbol at press 'K' (non-evil users must press 'C-c c k').
;; This will open documentation for it, including demos of how they are used.
;; Alternatively, use `C-h o' to look up a symbol (functions, variables, faces,
;; etc).
;;
;; You can also try 'gd' (or 'C-c c d') to jump to their definition and see how
;; they are implemented.
(use-package git-gutter
  :hook (prog-mode . git-gutter-mode)
  :config
  (setq git-gutter:update-interval 0.02))

(global-display-line-numbers-mode)
