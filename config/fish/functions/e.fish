function e --description 'Open a terminal-frame Emacs via emacsclient'
  command emacsclient -nw -s /run/user/1000/emacs/server $argv
end
