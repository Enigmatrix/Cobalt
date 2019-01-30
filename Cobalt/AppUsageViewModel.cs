using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data.Entities;
using DynamicData;
using ReactiveUI;

namespace Cobalt
{
    public class AppUsageViewModel : ReactiveObject
    {
        //TODO add Cobalt.Common.UI with a custom Application
        private AppUsage au;

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
