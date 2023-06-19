using Cobalt.Common.Utils;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialogView : DialogViewBase
{
    public AddAlertDialogView()
    {
        this.WhenActivated(() =>
        {
            var vm = (AddAlertDialogViewModel)DataContext!;
            var addTagInteractionCleanup = vm.AddTagInteraction.RegisterHandler(async ctx =>
            {
                var tag = await ctx.Input.ShowDialog<Unit, TagViewModel, AddTagDialogView>();
                if (tag == null) return;
                ctx.SetOutput(tag);
            });

            return () => { addTagInteractionCleanup(); };
        });

        InitializeComponent();
    }
}