using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Runtime.CompilerServices;
using System.Windows.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.UI.Util;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Views.Dialogs
{
    /// <summary>
    ///     Interaction logic for SelectAppsDialog.xaml
    /// </summary>
    public partial class SelectAppsDialog : INotifyPropertyChanged
    {
        private string _appFilter = "";
        private ICollectionView _apps;

        public SelectAppsDialog()
        {
            InitializeComponent();
        }

        public ICollectionView Apps
        {
            get => _apps;
            set
            {
                _apps = value;
                OnPropertyChanged();
            }
        }

        public string AppFilter
        {
            get => _appFilter;
            set
            {
                _appFilter = value;
                Apps?.Refresh();
                OnPropertyChanged();
            }
        }

        public event PropertyChangedEventHandler PropertyChanged;

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
            return StrContains(name, AppFilter) || StrContains(app.Path, AppFilter);
        }

        private bool StrContains(string n1, string n2)
        {
            return n1?.IndexOf(n2, StringComparison.OrdinalIgnoreCase) >= 0;
        }

        protected virtual void OnPropertyChanged([CallerMemberName] string propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}