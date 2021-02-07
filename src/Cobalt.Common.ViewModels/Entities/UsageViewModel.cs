using System;
using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.ViewModels.Entities
{
    public class UsageViewModel : EntityViewModelBase<Usage>
    {
        public UsageViewModel(Usage usage, IEntityManager manager) : base(usage, manager)
        {
            Id = usage.Id;
            Start = usage.Start;
            End = usage.End;
            DuringIdle = usage.DuringIdle;
            SessionId = usage.SessionId;
        }

        public DateTime Start { get; }
        public DateTime End { get; }
        public bool DuringIdle { get; }
        public long SessionId { get; }
        public SessionViewModel Session => Manager.GetSession(SessionId);

        public TimeSpan Duration => End - Start;
    }
}