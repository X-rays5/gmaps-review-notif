using discord_bot.discord;

namespace discord_bot;

public static class Init
{
    public static async Task Connect()
    {
        var client = new Client();
        await client.Connect();
    }
}
