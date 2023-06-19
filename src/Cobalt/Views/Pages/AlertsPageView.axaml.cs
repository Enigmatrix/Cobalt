using Avalonia.Controls;
using Cobalt.Common.Utils;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Pages;
using Cobalt.Views.Dialogs;

namespace Cobalt.Views.Pages;

public partial class AlertsPageView : UserControl
{
    public AlertsPageView()
    {
        this.WhenActivated(() =>
        {
            var vm = (AlertsPageViewModel)DataContext!;
            var addTagInteractionCleanup = vm.AddAlertInteraction.RegisterHandler(async ctx =>
            {
                var alert = await ctx.Input.ShowDialog<Unit, AlertViewModel, AddAlertDialogView>();
                if (alert == null) return;
                ctx.SetOutput(alert);
            });

            return () => { addTagInteractionCleanup(); };
        });

        InitializeComponent();
    }
}