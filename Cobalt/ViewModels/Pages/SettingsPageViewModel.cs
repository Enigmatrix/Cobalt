using System.Collections.Generic;
using System.ComponentModel;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Media;
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
            var swatches = new SwatchesProvider().Swatches.ToArray();
            MainHues = swatches.Select(x => x.ExemplarHue?.Color).Where(x => x!= null).Select(x => x.Value);
            AccentHues = swatches.Select(x => x.AccentExemplarHue?.Color).Where(x => x!= null).Select(x => x.Value);

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

        public IEnumerable<Color> MainHues { get; }
        public IEnumerable<Color> AccentHues { get; }

        public ISettingsService SettingsService { get; set; }

    }
}