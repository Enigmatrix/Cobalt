﻿using System.ComponentModel.DataAnnotations;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data.Entities;

[PrimaryKey(nameof(Guid), nameof(Version))]
public class Reminder : IEntity
{
    public required Alert Alert { get; set; }

    [Range(0.0, 1.0)] public double Threshold { get; set; }

    public required string Message { get; set; }
    public List<ReminderEvent> ReminderEvents { get; } = new();

    public long Version { get; set; }

    // can't autoincrement on integer partial keys, so use random guid instead
    public Guid Guid { get; set; } = Guid.NewGuid();

    public long Id => HashCode.Combine(Guid, Version);

    public Reminder Clone()
    {
        return new Reminder
        {
            Alert = Alert,
            Threshold = Threshold,
            Message = Message,
            Version = Version,
            Guid = Guid
        };
    }
}