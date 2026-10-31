Source tapes/common.tape
Set Height 880
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
Sleep 5s
Hide
Type "clear"
Enter
Sleep 300ms
Show
Type "sqlclean fix"
Sleep 300ms
Enter
Wait@60s /\$\s*$/
Sleep 1500ms
Type "cat models/orders.sql"
Enter
Sleep 5s
Hide
Type "clear"
Enter
Sleep 300ms
Show
Type "sqlclean fix --unsafe-fixes"
Sleep 300ms
Enter
Wait@60s /\$\s*$/
Sleep 1500ms
Type "cat models/orders.sql"
Enter
Sleep 1s
Screenshot out/check-fix.png
Sleep 5s
