using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Drawing;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Data;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Xml.Linq;
using ColorThiefDotNet;
using LiveCharts.Wpf;
using static System.Convert;
using Color = System.Windows.Media.Color;
using DColor = System.Drawing.Color;

namespace Cobalt.Common.UI.Converters
{
    public class ExePathToIconConverter : IValueConverter
    {
        private static readonly Dictionary<string, ImageSource> _iconMapper = new Dictionary<string, ImageSource>();

        public static readonly Dictionary<string, SolidColorBrush> ColorMapper =
            new Dictionary<string, SolidColorBrush>();

        private static readonly ColorThief _colorResolver = new ColorThief();

        public object Convert(object value, Type targetType, object parameter, CultureInfo culture)
        {
            ImageSource Get(string path)
            {
                try
                {
                    return IsNotModernApp(path) ? GetNormalAppIcon(path) : GetModernAppIcon(path);
                }
                catch (FileNotFoundException)
                {
                    return null;
                }
            }

            if (!(value is string pathStr)) return null;
            if (!_iconMapper.ContainsKey(pathStr)) _iconMapper[pathStr] = Get(pathStr);
            return _iconMapper[pathStr];
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        public static event Action<string> ColorGotten = delegate { };

        private static void Register(string path, Bitmap bitmap)
        {
            var otherCol = _colorResolver.GetColor(bitmap, ignoreWhite: false).Color;
            var netColor = DColor.FromArgb(otherCol.A, otherCol.R, otherCol.G, otherCol.B);
            ColorToHsv(netColor, out var h, out var s, out var v);
            var col = ColorFromHsv(360 - h, s, v);
            ColorMapper[path] = new SolidColorBrush(col);
            ColorGotten(path);
        }

        public static void ColorToHsv(DColor color, out double hue, out double saturation,
            out double value)
        {
            int max = Math.Max(color.R, Math.Max(color.G, color.B));
            int min = Math.Min(color.R, Math.Min(color.G, color.B));

            hue = color.GetHue();
            saturation = max == 0 ? 0 : 1d - 1d * min / max;
            value = max / 255d;
        }

        public static Color ColorFromHsv(double hue, double saturation, double value)
        {
            var hi = ToInt32(Math.Floor(hue / 60)) % 6;
            var f = hue / 60 - Math.Floor(hue / 60);

            value = value * 255;
            var v = ToByte(value);
            var p = ToByte(value * (1 - saturation));
            var q = ToByte(value * (1 - f * saturation));
            var t = ToByte(value * (1 - (1 - f) * saturation));

            switch (hi)
            {
                case 0:
                    return Color.FromArgb(255, v, t, p);
                case 1:
                    return Color.FromArgb(255, q, v, p);
                case 2:
                    return Color.FromArgb(255, p, v, t);
                case 3:
                    return Color.FromArgb(255, p, q, v);
                case 4:
                    return Color.FromArgb(255, t, p, v);
                default:
                    return Color.FromArgb(255, v, p, q);
            }
        }

        private static ImageSource GetModernAppIcon(string path)
        {
            //TODO cleanup all these IO reads with exception handling?
            //TODO better XML handling
            var directory = Path.GetDirectoryName(path);
            var exeName = Path.GetFileName(path);
            string imagePath = null;

            if (!Directory.Exists(directory)) return null;

            using (var fs = File.OpenRead(Path.Combine(directory, "AppxManifest.xml")))
            {
                var manifest = XDocument.Load(fs);

                var applicationNodes = manifest.Root?.Descendants()
                    .Where(x => x.Name.LocalName == "Application");

                var applicationNode = applicationNodes?
                    .Single(app => app.Attribute(XName.Get("Executable"))?.Value == exeName);

                var visualElements = applicationNode?.Elements()
                    .FirstOrDefault(x => x.Name.LocalName == "VisualElements");

                var imageRelPath =
                    //get the 44x44 (its usually the default)
                    visualElements?.Attribute(XName.Get("Square44x44Logo"))?.Value ??
                    //last is usually the smallest
                    visualElements?.Attributes().LastOrDefault(x => x.Name.LocalName.Contains("Logo"))
                        ?.Value;

                if (imageRelPath == null)
                    return null;

                foreach (var logoFile in Directory.GetFiles(
                    Path.Combine(directory,
                        Path.GetDirectoryName(imageRelPath) ?? throw new InvalidOperationException()),
                    //usually the file also comes with a scale e.g. Logo.scale-100.jpg. We just get the first one
                    Path.GetFileNameWithoutExtension(imageRelPath) + "*" + Path.GetExtension(imageRelPath)))
                {
                    imagePath = logoFile;
                    break;
                }
            }

            var bitmapImage = File.Exists(imagePath) ? new BitmapImage(new Uri($@"file:/{imagePath}")) : null;
            if (bitmapImage == null) return null;
            bitmapImage.DownloadCompleted += (o, e) =>
            {
                using (var outStream = new MemoryStream())
                {
                    var enc = new BmpBitmapEncoder();
                    enc.Frames.Add(BitmapFrame.Create(bitmapImage));
                    enc.Save(outStream);
                    var bitmap = new Bitmap(outStream);
                    Register(path, bitmap);
                }
            };
            return bitmapImage;
        }

        private static ImageSource GetNormalAppIcon(string path)
        {
            return ToImageSource(Icon.ExtractAssociatedIcon(path), path);
        }

        private static bool IsNotModernApp(string path)
        {
            return !path.Contains(@"Program Files\WindowsApps");
        }

        [DllImport("gdi32.dll", SetLastError = true)]
        private static extern bool DeleteObject(IntPtr hObject);

        private static ImageSource ToImageSource(Icon icon, string path)
        {
            var bitmap = icon.ToBitmap();
            var hBitmap = bitmap.GetHbitmap();
            Register(path, bitmap);

            var wpfBitmap = Imaging.CreateBitmapSourceFromHBitmap(
                hBitmap,
                IntPtr.Zero,
                Int32Rect.Empty,
                BitmapSizeOptions.FromEmptyOptions());

            if (!DeleteObject(hBitmap))
                throw new Win32Exception();

            return wpfBitmap;
        }
    }

    public static class ColorSync
    {
        public static T BindFill<T>(this T ser, string p)
            where T : Series
        {
            void ColorGet(string path)
            {
                if (path != p) return;
                ser.Fill = ExePathToIconConverter.ColorMapper[p];
                ExePathToIconConverter.ColorGotten -= ColorGet;
            }

            if (ExePathToIconConverter.ColorMapper.ContainsKey(p))
                ser.Fill = ExePathToIconConverter.ColorMapper[p];
            else ExePathToIconConverter.ColorGotten += ColorGet;
            return ser;
        }
    }
}