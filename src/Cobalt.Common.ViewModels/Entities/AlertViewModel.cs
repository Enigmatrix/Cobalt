#pragma warning disable CS8618 // Non-nullable field must contain a non-null value when exiting constructor. Consider declaring as nullable.
using System;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Entities
{
    public class TargetViewModel
    {
        public sealed class App : TargetViewModel
        {
            public App(AppViewModel val)
            {
                Value = val;
            }

            public AppViewModel Value { get; }
        }

        public sealed class Tag : TargetViewModel
        {
            public Tag(TagViewModel val)
            {
                Value = val;
            }

            public TagViewModel Value { get; }
        }
    }

    public class AlertViewModel : MutableEntityViewModelBase<Alert>
    {
        public AlertViewModel(Alert alert, IEntityManager manager, IDatabase db) : base(alert, manager, db)
        {
            Id = alert.Id;
            UpdateFromEntity(alert);
        }

        [Reactive] public TargetViewModel Target { get; set; }
        [Reactive] public TimeFrame TimeFrame { get; set; }
        [Reactive] public TimeSpan UsageLimit { get; set; }
        [Reactive] public Reaction ExceededReaction { get; set; }

        public sealed override void UpdateFromEntity(Alert alert)
        {
            Target = alert.Target switch
            {
                Target.App app => new TargetViewModel.App(Manager.GetApp(app.AppId)),
                Target.Tag tag => new TargetViewModel.Tag(Manager.GetTag(tag.TagId)),
                _ => throw new ArgumentOutOfRangeException(nameof(alert))
            };
            TimeFrame = alert.TimeFrame;
            UsageLimit = alert.UsageLimit;
            ExceededReaction = alert.ExceededReaction;
        }

        public override void Save()
        {
            // TODO
        }
    }
}
#pragma warning restore CS8618