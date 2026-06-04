using System;
using System.Collections.Generic;
using System.IO;
using Rocket.Core.Plugins;
using SDG.Unturned;
using Steamworks;
using UnityEngine;

namespace UnturnedServerManagerBridge
{
    public sealed class BridgePlugin : RocketPlugin
    {
        private const string QueueRelativePath = "Servers/{0}/Rocket/Plugins/UnturnedServerManagerBridge/commands.queue";
        private const string ProcessingFileName = "commands.processing";
        private readonly object queueLock = new object();
        private string queueDirectory;
        private string queuePath;
        private string processingPath;
        private string readyPath;
        private float nextPollTime;

        protected override void Load()
        {
            queuePath = Path.Combine(ReadWrite.PATH, string.Format(QueueRelativePath, Provider.serverID));
            queueDirectory = Path.GetDirectoryName(queuePath);
            System.IO.Directory.CreateDirectory(queueDirectory);
            processingPath = Path.Combine(queueDirectory, ProcessingFileName);
            readyPath = Path.Combine(queueDirectory, "bridge.ready");
            if (!System.IO.File.Exists(queuePath))
            {
                System.IO.File.WriteAllText(queuePath, string.Empty);
            }
            System.IO.File.WriteAllText(readyPath, DateTime.UtcNow.ToString("O"));
            CommandWindow.Log("[USM Bridge] Local command bridge loaded.");
        }

        protected override void Unload()
        {
            if (!string.IsNullOrEmpty(readyPath))
            {
                try
                {
                    System.IO.File.Delete(readyPath);
                }
                catch
                {
                }
            }
            CommandWindow.Log("[USM Bridge] Local command bridge unloaded.");
        }

        private void Update()
        {
            if (Time.realtimeSinceStartup < nextPollTime)
            {
                return;
            }

            nextPollTime = Time.realtimeSinceStartup + 0.25f;
            ProcessQueue();
        }

        private void ProcessQueue()
        {
            if (string.IsNullOrEmpty(queuePath) || string.IsNullOrEmpty(processingPath))
            {
                return;
            }

            List<string> commands;
            lock (queueLock)
            {
                commands = TakeQueuedCommands();
            }

            foreach (string command in commands)
            {
                try
                {
                    CommandWindow.Log("[USM Bridge] Executing local command: " + command);
                    Commander.execute(CSteamID.Nil, command);
                }
                catch (Exception ex)
                {
                    CommandWindow.LogError("[USM Bridge] Command failed: " + command + " (" + ex.Message + ")");
                }
            }
        }

        private List<string> TakeQueuedCommands()
        {
            if (System.IO.File.Exists(processingPath))
            {
                return ReadAndDeleteProcessingFile(processingPath);
            }

            if (!System.IO.File.Exists(queuePath))
            {
                return new List<string>();
            }

            try
            {
                if (new FileInfo(queuePath).Length == 0)
                {
                    return new List<string>();
                }
                System.IO.File.Move(queuePath, processingPath);
            }
            catch (Exception ex)
            {
                CommandWindow.LogError("[USM Bridge] Failed to claim command queue: " + ex.Message);
                return new List<string>();
            }

            return ReadAndDeleteProcessingFile(processingPath);
        }

        private List<string> ReadAndDeleteProcessingFile(string path)
        {
            List<string> commands = new List<string>();
            try
            {
                using (FileStream stream = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.Read))
                using (StreamReader reader = new StreamReader(stream))
                {
                    string line;
                    while ((line = reader.ReadLine()) != null)
                    {
                        line = line.Trim();
                        if (line.Length > 0)
                        {
                            commands.Add(line);
                        }
                    }
                }
                System.IO.File.Delete(path);
            }
            catch (Exception ex)
            {
                CommandWindow.LogError("[USM Bridge] Failed to read command queue: " + ex.Message);
                return new List<string>();
            }

            return commands;
        }
    }
}
