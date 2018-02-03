﻿using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.UI;
using Cobalt.Properties;
using MahApps.Metro;
using MaterialDesignColors;
using MaterialDesignThemes.Wpf;

namespace Cobalt.ViewModels.Utils
{
    public interface ISettingsService : INotifyPropertyChanged
    {
        bool IsDark { get; set; }
    }
    public class SettingsService : NotifyPropertyChanged, IDisposable, ISettingsService
    {
        private readonly Settings _settings;
        private readonly PaletteHelper _palette;
        private readonly IDisposable _saveTracker;

        public SettingsService()
        {
            _settings = Settings.Default;
            _palette = new PaletteHelper();

            //reassign to trigger change
            IsDark = _settings.IsDark;

            _saveTracker = Observable.FromEventPattern<PropertyChangedEventHandler, PropertyChangedEventArgs>(
                    handler => handler.Invoke, h => PropertyChanged += h, h => PropertyChanged -= h)
                .Throttle(TimeSpan.FromMilliseconds(150))
                .Subscribe(x =>
                {
                    _settings.Save();
                });
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

        public void Dispose()
        {
            _saveTracker.Dispose();
        }
    }

}