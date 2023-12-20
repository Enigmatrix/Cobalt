#if DEBUG
using System.Collections.Generic;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Markup.Xaml.Templates;
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Pages;

namespace Cobalt.Views.Pages;

public partial class ExperimentsPage : ReactiveUserControl<ExperimentsPageViewModel>
{
    public ExperimentsPage()
    {
        InitializeComponent();
    }

    private async Task<IEnumerable<object>> WhatAsyncPopulator(string? s, CancellationToken token)
    {
        return await ViewModel!.GetApps(s);
    }
}
#endif