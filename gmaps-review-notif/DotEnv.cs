using log4net;

namespace gmaps_review_notif;

using System;
using System.IO;

public static class DotEnv
{
    private static readonly ILog LOG = LogManager.GetLogger(typeof(DotEnv));

    public static void Load(string filePath)
    {
        if (!File.Exists(filePath))
            return;

        foreach (var line in File.ReadAllLines(filePath))
        {
            var parts = line.Split('=', StringSplitOptions.RemoveEmptyEntries);

            if (parts.Length != 2)
                continue;

            LOG.Info($"Loading ${parts[0]}");
            Environment.SetEnvironmentVariable(parts[0], parts[1]);
        }
    }
}
