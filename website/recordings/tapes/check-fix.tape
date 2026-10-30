Source tapes/common.tape
Set Height 820
Output out/check-fix.gif
Output out/check-fix.mp4
Hide
Type "cd scratch/check-fix && export PS1='$ ' && clear"
Enter
Sleep 500ms
Show
Type "cat models/orders.sql"
Enter
Sleep 1500ms
Type "sqlclean check"
Sleep 300ms
Enter
Wait@60s /\$\s*$/
Sleep 3s
Type "clear && sqlclean fix"
Sleep 300ms
Enter
Wait@60s /\$\s*$/
Sleep 1500ms
Type "cat models/orders.sql"
Enter
Sleep 1s
Screenshot out/check-fix.png
Sleep 8s
