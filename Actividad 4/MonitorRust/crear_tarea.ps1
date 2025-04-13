$action = New-ScheduledTaskAction -Execute "C:\Users\Compu\Desktop\MonitorRust\monitor.exe"

$trigger1 = New-ScheduledTaskTrigger -AtStartup
$trigger2 = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 10) -RepetitionDuration (New-TimeSpan -Days 2)

Register-ScheduledTask -TaskName "MonitoreoRust" `
    -Action $action `
    -Trigger @($trigger1, $trigger2) `
    -RunLevel Highest