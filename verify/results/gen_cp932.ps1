$cp = [int[]](0x30a8,0x30e9,0x30fc,0x003a,0x0020,0x5909,0x63db,0x306b,0x5931,0x6557,0x3057,0x307e,0x3057,0x305f)
$s = -join ($cp | ForEach-Object { [char]$_ })
$b = [System.Text.Encoding]::GetEncoding(932).GetBytes($s + "`r`n")
[System.IO.File]::WriteAllBytes("C:\devin\kasuga_converter\verify\results\verify116_err.bin", $b)
