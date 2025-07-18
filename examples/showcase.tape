# https://github.com/charmbracelet/vhs
Output showcase.gif

# setup
Set FontSize 16
Set Margin 10
Set Width 640
Set Height 460

# wait for shell to boot up
Sleep 100ms

# run
Hide
Type@10ms "cargo r -F vhs_keybinds"
Enter
Show
Sleep 2500ms

# play dream speedrun music
Type "t"
Sleep 400ms
Type@120ms "dream"
Sleep 500ms
Enter

Sleep 2500ms

# toggle shit mic mode 4 times
Enter
Sleep 400ms
Enter
Sleep 410ms
Enter
Sleep 390ms
Enter

Sleep 1100ms

# stop playing dream speedrun music
Type "s"

Sleep 1100ms

# enable random audio triggering
Down
Sleep 300ms
Enter

Sleep 2000ms

# disable random audio triggering
Enter

Sleep 1100ms

# exit
Down
Sleep 330ms
Down
Sleep 310ms
Down
Sleep 320ms
Down
Sleep 640ms
Enter
