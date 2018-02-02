using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.IoC;
using Cobalt.ViewModels.Utils;
using MaterialDesignColors;
using MaterialDesignThemes.Wpf;

namespace Cobalt.ViewModels.Pages
{
    public class SettingsPageViewModel : PageViewModel
    {
        private readonly ISettingsService _settings;

        public SettingsPageViewModel(IResourceScope scope, ISettingsService settings) : base(scope)
        {
            _settings = settings;
            Swatches = new SwatchesProvider().Swatches;
            _settings.PropertyChanged += SettingsPropertyChanged;
        }

        private void SettingsPropertyChanged(object sender, PropertyChangedEventArgs e)
        {
            NotifyOfPropertyChange(e.PropertyName);
        }

        public bool IsDark
        {
            get => _settings.IsDark;
            set => _settings.IsDark = value;
        }

        public IEnumerable<Swatch> Swatches { get; }
        public ISettingsService SettingsService { get; set; }

    }
}