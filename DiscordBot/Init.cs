using DiscordBot.Discord;

namespace DiscordBot;

public static class Init
{
    public static async Task Connect()
    {
        var client = new Client();
        await client.Connect();
    }
}
