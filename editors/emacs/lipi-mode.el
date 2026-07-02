;;; lipi-mode.el --- Major mode for LIPI (Devanagari programming language) -*- lexical-binding: t; -*-

;; Simple major mode: keyword highlighting, comments, indentation, and an
;; optional LSP hook (uses `lipi lsp`). Load with:
;;   (add-to-list 'load-path "/path/to/editors/emacs")
;;   (require 'lipi-mode)

(defvar lipi-keywords
  '("विधि" "फल" "यदि" "अन्यथा" "अन्यथा यदि" "जब तक" "के लिए" "में" "बार करो"
    "बंद करो" "अगला" "वर्ग" "यह" "बताओ" "लिखो" "है" "और" "या" "नहीं"
    "कोशिश" "पकड़ो" "फेंको" "जाँचो" "लाम्डा" "आयात" "उत्पन्न" "मिलाओ"
    "सत्य" "असत्य" "शून्य" "वैश्विक" "स्थिर" "तो" "से अधिक" "से कम" "बराबर"
    "साझा" "सार" "साथ" "के_रूप_में" "अभिलेख" "में_है" "नहीं_है")
  "LIPI keywords.")

(defvar lipi-font-lock-keywords
  (list
   (cons (regexp-opt lipi-keywords 'words) 'font-lock-keyword-face)
   '("\\_<भारत\\.[^ (]+" . font-lock-type-face)
   '("विधि \\([^ (]+\\)" 1 font-lock-function-name-face)))

(defvar lipi-mode-syntax-table
  (let ((st (make-syntax-table)))
    (modify-syntax-entry ?# "<" st)     ; # starts a comment
    (modify-syntax-entry ?\n ">" st)    ; newline ends it
    (modify-syntax-entry ?\" "\"" st)
    st)
  "Syntax table for `lipi-mode'.")

;;;###autoload
(define-derived-mode lipi-mode prog-mode "LIPI"
  "Major mode for editing LIPI source."
  :syntax-table lipi-mode-syntax-table
  (setq-local font-lock-defaults '(lipi-font-lock-keywords))
  (setq-local comment-start "# ")
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width 4)
  ;; Optional LSP: if eglot is available, register the LIPI server.
  (when (featurep 'eglot)
    (add-to-list 'eglot-server-programs '(lipi-mode . ("lipi" "lsp")))))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.swami\\'" . lipi-mode))
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.\\(roman\\|vani\\)\\'" . lipi-mode))

(provide 'lipi-mode)
;;; lipi-mode.el ends here
