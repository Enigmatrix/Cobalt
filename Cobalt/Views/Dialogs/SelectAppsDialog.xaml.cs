using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Converters;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.Dialogs
{
    /// <summary>
    /// Interaction logic for SelectAppsDialog.xaml
    /// </summary>
    public partial class SelectAppsDialog : INotifyPropertyChanged
    {
        private ICollectionView _apps;
        private string _appFilter = "";

        public SelectAppsDialog()
        {
            InitializeComponent();
        }

        public ICollectionView Apps
        {
            get => _apps;
            set { _apps = value; OnPropertyChanged(); }
        }

        public string AppFilter
        {
            get => _appFilter;
            set { _appFilter = value;
                Apps?.Refresh();
                OnPropertyChanged(); }
        }

        public override void Prepare(object[] args)
        {
            var apps = (IObservable<AppViewModel>) args[0];
            var res = (IResourceScope) args[1];
            var appColl = new ObservableCollection<AppViewModel>();
            apps.Subscribe(x => appColl.Add(x)).ManagedBy(res);
            Apps = CollectionViewSource.GetDefaultView(appColl);
            Apps.Filter = AppMatch;
            DataContext = this;
        }

        private bool AppMatch(object obj)
        {
            var app = (AppViewModel) obj;
            if (AppFilter == "") return true;
            var name = app.Name ?? AppResourceCache.Instance.GetName(app.Path);
            return  StrContains(name, AppFilter) || StrContains(app.Path, AppFilter);
        }

        private bool StrContains(string n1, string n2)
        {
            return n1?.IndexOf(n2, StringComparison.OrdinalIgnoreCase) >= 0;
        }

        public event PropertyChangedEventHandler PropertyChanged;

        protected virtual void OnPropertyChanged([CallerMemberName] string propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
