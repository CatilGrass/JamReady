using System;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;

namespace JamReadyGui._Utils.Linker;

public static class JamLinker
{
    private static JamLinkerClient _current = new();
    public static JamLinkerClient Current => _current;

    public static string Address { get; set; } = "127.0.0.1";
    public static int Port { get; set; } = 5012;

    public static bool Online
    {
        get => Current.Connected;
        set
        {
            var current = Online;
            if (value == current) return;
            if (value && !current)
                _ = StartListen();
            else if (!value && current)
                StopListen();
        }
    }

    private static async Task StartListen()
    {
        var client = JamLinkerClient.Setup(Address, Port);
        var connected = await client.ConnectAsync();
        if (connected)
            _current = client;
    }

    private static void StopListen()
    {
        if (Online)
            _current.Disconnect();
    }
    
    public class JamLinkerClient
    {
        private TcpClient? _client;
        private NetworkStream? _stream;
        private bool _connected;
        
        private string _host = "127.0.0.1";
        private int _port;

        public bool Connected => _connected &&
                                 _client is { Connected: true } &&
                                 _stream is { Socket.Connected: true };

        public static JamLinkerClient Setup(string host, int port)
        {
            var client = new JamLinkerClient
            {
                _host = host,
                _port = port
            };
            return client;
        }
    
        public async Task<bool> ConnectAsync()
        {
            try
            {
                _client = new TcpClient();
                await _client.ConnectAsync(_host, _port);
                _stream = _client.GetStream();
                _connected = true;
                return true;
            }
            catch (Exception)
            {
                return false;
            }
        }
    
        public async Task<string> SendCommandAsync(string command)
        {
            if (! Connected) return "";
            
            try
            {
                if (!command.EndsWith("\n"))
                    command += "\n";
                    
                byte[] requestBytes = Encoding.UTF8.GetBytes(command);
                await _stream.WriteAsync(requestBytes, 0, requestBytes.Length);
    
                byte[] buffer = new byte[2048];
                int bytesRead = await _stream.ReadAsync(buffer, 0, buffer.Length);
                
                if (bytesRead > 0)
                {
                    string response = Encoding.UTF8.GetString(buffer, 0, bytesRead);
                    return response;
                }
                
                return string.Empty;
            }
            catch (Exception)
            {
                return string.Empty;
            }
        }
    
        public void Disconnect()
        {
            _connected = false;
            _stream?.Close();
            _client?.Close();
        }
    }
}