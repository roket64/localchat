$server = "127.0.0.1"
$port = "8080"
$message = "test_message"

# create tcp connection to the server
$client = New-Object System.Net.Sockets.TcpClient($server, $port)
$stream = $client.GetStream()

# convert string into bytes
$bytes = [System.Text.Encoding]::ASCII.GetBytes($message)

# send bytes
$stream.Write($bytes, 0, $bytes.Length)

# close stream and client
$stream.Close()
$client.Close()

Write-Output "Message sent to $server on port $port"
