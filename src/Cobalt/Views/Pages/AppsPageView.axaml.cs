using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Net.Http;
using Avalonia.Controls;
using Cobalt.Common.Analysis;
using Cobalt.Common.Analysis.Graphs;
using Cobalt.Common.ViewModels.Entities;
using LiveChartsCore;
using LiveChartsCore.Kernel;
using LiveChartsCore.Kernel.Sketches;

namespace Cobalt.Views.Pages;

public partial class AppsPageView : UserControl
{
    public AppsPageView()
    {
        var stat = new Statistic<WithDuration<AppViewModel>>(
            () => new[]
            {
                new WithDuration<AppViewModel>(new AppViewModel(null!, null!)
                {
                    Name = "Chrome",
                    Color = "#89dceb",
                    Icon = Fetch(
                        "https://upload.wikimedia.org/wikipedia/commons/8/87/Google_Chrome_icon_%282011%29.png")
                }, TimeSpan.FromHours(20)),
                new WithDuration<AppViewModel>(new AppViewModel(null!, null!)
                {
                    Name = "League of Legends",
                    Color = "#fab387",
                    Icon = Fetch(
                        "https://icon-library.com/images/league-of-legends-icon-transparent/league-of-legends-icon-transparent-0.jpg")
                }, TimeSpan.FromHours(10)),
                new WithDuration<AppViewModel>(new AppViewModel(null!, null!)
                {
                    Name = "Cobalt",
                    Color = "#f2cdcd",
                    Icon = Fetch("https://raw.githubusercontent.com/Enigmatrix/Cobalt/master/images/icon_512.png")
                }, TimeSpan.FromHours(5))
            }.AsQueryable(), new PieGraph<WithDuration<AppViewModel>>());

        // TODO this needs to be sleeker
        stat.Graph.SetData(stat.Query.Execute());
        Series = stat.Graph.Series;

        InitializeComponent();
    }

    public ObservableCollection<ISeries> Series { get; set; } = new();

    private Stream Fetch(string url)
    {
        var client = new HttpClient();
        using var stream = client.GetStreamAsync(url).Result;
        var mem = new MemoryStream();
        stream.CopyTo(mem);
        mem.Seek(0, SeekOrigin.Begin);
        return mem;
    }


    private void PieChart_OnChartPointPointerDown(IChartView chart, ChartPoint? point)
    {
        if (point?.Context.DataSource is WithDuration<AppViewModel> withDur)
            withDur.Duration += TimeSpan.FromHours(1);
    }
}