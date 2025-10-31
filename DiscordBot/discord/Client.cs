using System.Reflection;
using Discord;
using Discord.Net;
using Discord.WebSocket;
using log4net;
using Newtonsoft.Json;

namespace DiscordBot.Discord;

public class Client
{
    private static readonly ILog LOG = LogManager.GetLogger(typeof(Client));

    private static DiscordSocketClient _client;

    private readonly Dictionary<string, Delegate> _commands = new();

    public async Task Connect()
    {
        _client = new DiscordSocketClient(new DiscordSocketConfig
        {
            LogLevel = LogSeverity.Debug
        });

        _client.Log += Log;
        _client.Ready += ClientReady;

        await _client.LoginAsync(TokenType.Bot, Environment.GetEnvironmentVariable("gmaps_review_notif"));
        await _client.StartAsync();

        _client.SlashCommandExecuted += SlashCommandHandler;
    }

    private async Task ClientReady()
    {
        LOG.Info("Init ready");

        var commands = FindSlashCommandHandlers();
        var existingCommands = await _client.GetGlobalApplicationCommandsAsync();

        try
        {
            foreach (var command in commands)
            {
                var properties = command.GetMethod("RegisterCommand").Invoke(null, null) as SlashCommandProperties;

                var existing = existingCommands.FirstOrDefault(c => c.Name == properties.Name.Value);

                if (existing == null || !AreCommandsEqual(existing, properties))
                {
                    await _client.CreateGlobalApplicationCommandAsync(properties);
                    LOG.Info($"Created/Updated command: {properties.Name}");
                }

                var handleMethod = command.GetMethod("HandleCommand", BindingFlags.Static | BindingFlags.Public);
                var handleDelegate = (Func<SocketSlashCommand, Task>)Delegate.CreateDelegate(typeof(Func<SocketSlashCommand, Task>), handleMethod);
                _commands[properties.Name.Value] = handleDelegate;
            }
        }
        catch (HttpException ex)
        {
            var json = JsonConvert.SerializeObject(ex.Errors, Formatting.Indented);
            LOG.ErrorFormat("Failed to create/update global application command: {0}", json);
        }
    }

    private bool AreCommandsEqual(SocketApplicationCommand? existing, SlashCommandProperties? local)
    {
        if (existing == null) return false;
        if (local == null) return false;

        if (existing.Description != local.Description.Value) return false;
        if ((existing.Options?.Count ?? 0) != (local.Options.Value.Count)) return false;

        if (existing.Options != null && local.Options.IsSpecified)
        {
            for (int i = 0; i < existing.Options.Count; i++)
            {
                var eOpt = existing.Options.ElementAt(i);
                var lOpt = local.Options.Value[i];
                if (eOpt.Name != lOpt.Name || eOpt.Description != lOpt.Description || eOpt.Type != lOpt.Type)
                    return false;
            }
        }

        return true;
    }

    private static IEnumerable<Type> FindSlashCommandHandlers()
    {
        Assembly assembly = Assembly.GetExecutingAssembly();
        var types = assembly.GetTypes().Where(t => t.Namespace == "DiscordBot.Discord.Commands");
        return types.Where(t => t.GetInterfaces().Any(i => i == typeof(SlashCommandHandler)));
    }

    private async Task SlashCommandHandler(SocketSlashCommand command)
    {
        if (_commands.ContainsKey(command.Data.Name))
        {
            await (Task)_commands[command.Data.Name].DynamicInvoke(command);
            return;
        }

        await command.RespondAsync("Failed to find command handler");
    }

    private static Task Log(LogMessage msg)
    {
        switch(msg.Severity)
        {
            case LogSeverity.Critical:
                LOG.Fatal(msg.Message, msg.Exception);
                break;
            case LogSeverity.Error:
                LOG.Error(msg.Message, msg.Exception);
                break;
            case LogSeverity.Warning:
                LOG.Warn(msg.Message, msg.Exception);
                break;
            case LogSeverity.Info:
                LOG.Info(msg.Message, msg.Exception);
                break;
            case LogSeverity.Verbose:
            case LogSeverity.Debug:
                LOG.Debug(msg.Message, msg.Exception);
                break;
        }

        return Task.CompletedTask;
    }
}
