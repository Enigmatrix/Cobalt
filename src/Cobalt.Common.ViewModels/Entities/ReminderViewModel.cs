using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the Reminder Entity
/// </summary>
public partial class ReminderViewModel : EditableEntityViewModelBase<Reminder>
{
    [ObservableProperty] private string _message;
    [ObservableProperty] private double _threshold;

    public ReminderViewModel(Reminder entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(entity, entityCache, contexts)
    {
        Message = entity.Message;
        Threshold = entity.Threshold;
    }

    public override void UpdateEntity()
    {
        // We ignore updating Alert here, since that will never happen
        Entity.Message = Message;
        Entity.Threshold = Threshold;
    }
}