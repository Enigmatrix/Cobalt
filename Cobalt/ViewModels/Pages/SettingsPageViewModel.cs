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
            //todo propogate changes from isettingsservice to settingspageviewmodel
            _settings = settings;
            Swatches = new SwatchesProvider().Swatches;
        }

        public bool IsDark
        {
            get => _settings.IsDark;
            set => _settings.IsDark = value;
        }

        public IEnumerable<Swatch> Swatches { get; }
    }
}