using System;
using System.ComponentModel;
using System.Reactive.Linq;
using Cobalt.Common.UI;
using Cobalt.Common.Util;
using Cobalt.Properties;
using MaterialDesignThemes.Wpf;

namespace Cobalt.ViewModels.Utils
{
    public interface ISettingsService : INotifyPropertyChanged
    {
        bool IsDark { get; set; }
    }

    public class SettingsService : NotifyPropertyChanged, IDisposable, ISettingsService
    {
        private readonly PaletteHelper _palette;
        private readonly IDisposable _saveTracker;
        private readonly Settings _settings;

        public SettingsService()
        {
            _settings = Settings.Default;
            _palette = new PaletteHelper();

            //reassign to trigger change
            IsDark = _settings.IsDark;

            _saveTracker = this.PropertyChanges()
                .Throttle(TimeSpan.FromMilliseconds(150))
                .Subscribe(x => _settings.Save());
        }

        public void Dispose()
        {
            _saveTracker.Dispose();
        }

        public bool IsDark
        {
            get => _settings.IsDark;
            set
            {
                _settings.IsDark = value;
                _palette.SetLightDark(value);
                NotifyOfPropertyChange();
            }
        }
    }
}