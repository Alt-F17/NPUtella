Set WshShell = CreateObject("WScript.Shell")
Set Fso = CreateObject("Scripting.FileSystemObject")

ScriptDir = Fso.GetParentFolderName(WScript.ScriptFullName)
ProjectDir = ScriptDir
NativeExe = ProjectDir & "\nputella.exe"
LogPath = ProjectDir & "\nputella_launcher.log"

Sub LogLine(Message)
  Set LogFile = Fso.OpenTextFile(LogPath, 8, True)
  LogFile.WriteLine Now & " " & Message
  LogFile.Close
End Sub

LogLine "launcher started from " & WScript.ScriptFullName
LogLine "script dir " & ScriptDir
LogLine "project dir " & ProjectDir

If Fso.FileExists(NativeExe) Then
  LogLine "launching native exe " & NativeExe
  WshShell.Run """" & NativeExe & """", 0, False
Else
  LogLine "native exe missing at " & NativeExe
End If
