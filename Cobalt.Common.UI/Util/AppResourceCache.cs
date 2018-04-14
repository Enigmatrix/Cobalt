using System;
using System.Collections.Generic;
using System.IO;
using System.Reactive.Threading.Tasks;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using Cobalt.Common.UI.ViewModels;

namespace Cobalt.Common.UI.Util
{
    public class AppResourceCache
    {
        private readonly Dictionary<string, SolidColorBrush> _colorMapper = new Dictionary<string, SolidColorBrush>();
        private readonly Dictionary<string, ImageSource> _iconMapper = new Dictionary<string, ImageSource>();


        private readonly Random _rng = new Random();
        public static AppResourceCache Instance { get; } = new AppResourceCache();


        public SolidColorBrush GetColor(AppViewModel app)
        {
            if (!_colorMapper.ContainsKey(app.Path))
                _colorMapper[app.Path] =
                    new SolidColorBrush(app.Color == null
                        ? Colors.Transparent
                        : (Color) ColorConverter.ConvertFromString(app.Color));
            return _colorMapper[app.Path];
        }

        public ImageSource GetIcon(AppViewModel app)
        {
            if (!_iconMapper.ContainsKey(app.Path))
                _iconMapper[app.Path] = LoadImage(app.Icon?.ToTask().Result);
            return _iconMapper[app.Path];
        }

        private static BitmapImage LoadImage(byte[] imageData)
        {
            if (imageData == null || imageData.Length == 0) return null;
            var image = new BitmapImage();
            using (var mem = new MemoryStream(imageData))
            {
                mem.Position = 0;
                image.BeginInit();
                image.CreateOptions = BitmapCreateOptions.PreservePixelFormat;
                image.CacheOption = BitmapCacheOption.OnLoad;
                image.UriSource = null;
                image.StreamSource = mem;
                image.EndInit();
            }

            image.Freeze();
            return image;
        }
    }
}