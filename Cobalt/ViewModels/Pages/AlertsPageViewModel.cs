using Caliburn.Micro;
using Cobalt.Common.Analysis;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.UI.ViewModels;
using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.ViewModels.Pages
{
    public class AlertsPageViewModel : PageViewModel
    {
        private BindableCollection<AppAlertViewModel> _appAlerts;
        private BindableCollection<TagAlertViewModel> _tagAlerts;

        public IDbRepository Repository { get; }

        public static AlertsPageViewModel Test => new AlertsPageViewModel(null, null);

        private AppAlertViewModel mew = 
                    new AppAlertViewModel(new AppAlert{ 
                        App = new App { Id = 6, Path = @"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe" },
                        IsEnabled = true,
                        AlertAction = AlertAction.Kill,
                        ReminderOffset = TimeSpan.FromMinutes(5),
                        MaxDuration = TimeSpan.FromHours(2),
                        Range = new OnceAlertRange(){
                            StartTimestamp = DateTime.Now,
                            EndTimestamp = DateTime.Now.AddHours(1),
                        }
                    });

        public AlertsPageViewModel(IResourceScope scope, IDbRepository repo) : base(scope)
        {
            _appAlerts = new BindableCollection<AppAlertViewModel>(new[]{ 
                mew
                });
            _tagAlerts = new BindableCollection<TagAlertViewModel>();
            Repository = repo;
        }

        public BindableCollection<AppAlertViewModel> AppAlerts
        {
            get => _appAlerts;
            set => Set(ref _appAlerts, value);
        }

        public BindableCollection<TagAlertViewModel> TagAlerts
        {
            get => _tagAlerts;
            set => Set(ref _tagAlerts, value);
        }

        public void AddAppAlert()
        {
            AppAlerts.Add(mew);
        }

        protected override void OnActivate(IResourceScope resources)
        {
            var repo = resources.Resolve<IEntityStreamService>();
            repo.GetAlertChanges().Subscribe(x => {
                switch (x.ChangeType)
                {
                    case ChangeType.Add:
                        if(x.AssociatedEntity is AppAlert addAppAlert)
                            AppAlerts.Add(EnableSaving(new AppAlertViewModel(addAppAlert)));

                        else if(x.AssociatedEntity is TagAlert addTagAlert)
                            TagAlerts.Add(EnableSaving(new TagAlertViewModel(addTagAlert)));
                        break;

                    case ChangeType.Remove:
                        if(x.AssociatedEntity is AppAlert rmvAppAlert)
                            AppAlerts.Remove(DisableSaving(AppAlerts.Single(a => a.Id == rmvAppAlert.Id)));

                        else if(x.AssociatedEntity is TagAlert rmvTagAlert)
                            AppAlerts.Remove(DisableSaving(AppAlerts.Single(a => a.Id == rmvTagAlert.Id)));
                        break;

                    case ChangeType.Modify:
                        //todo in another life, if another cobalt is open, use this
                        if(x.AssociatedEntity is AppAlert modAppAlert) { }
                        else if(x.AssociatedEntity is TagAlert rmvTagAlert) { }
                        break;
                }
            }).ManageUsing(resources);
        }
        public T EnableSaving<T>(T alert) where T : AlertViewModel
        {
            alert.PropertyChanged += SaveAlert;
            return alert;
        }

        public T DisableSaving<T>(T alert) where T : AlertViewModel
        {
            alert.PropertyChanged -= SaveAlert;
            return alert;
        }

        private void SaveAlert(object sender, PropertyChangedEventArgs e)
        {
            //also transmit it to transmissionserver
            Repository.UpdateAlert((Alert)sender);
        }
    }
}
