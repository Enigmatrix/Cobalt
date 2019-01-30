using System;
using Cobalt.Common.Data.Entities;
using ReactiveUI;

namespace Cobalt
{
    public class AppUsageViewModel : ReactiveObject
    {
        //TODO add Cobalt.Common.UI with a custom Application
        private readonly AppUsage au;

        public AppUsageViewModel(AppUsage au)
        {
            this.au = au;
        }

        public string Path
        {
            get => au.App.Path;
            set
            {
                au.App.Path = value;
                this.RaisePropertyChanged();
            }
        }

        public Lazy<byte[]> Image => au.App.Icon;
    }
}