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

namespace Cobalt.Common.UI.Converters
{
    public class ExePathToIconConverter : IValueConverter
    {
        private readonly Dictionary<string, ImageSource> _iconMapper = new Dictionary<string, ImageSource>();

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

            var pathStr = value as string;
            if (pathStr is null) return null;
            if (!_iconMapper.ContainsKey(pathStr)) _iconMapper[pathStr] = Get(pathStr);
            return _iconMapper[pathStr];
        }

        public object ConvertBack(object value, Type targetType, object parameter, CultureInfo culture)
        {
            throw new NotImplementedException();
        }

        private static ImageSource GetModernAppIcon(string path)
        {
            var directory = Path.GetDirectoryName(path);
            var exeName = Path.GetFileName(path);
            string imagePath = null;
            using (var fs = File.OpenRead(Path.Combine(directory ?? throw new InvalidOperationException(),
                "AppxManifest.xml")))
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
            return new BitmapImage(new Uri($@"file:/{imagePath}"));
        }

        private static ImageSource GetNormalAppIcon(string path)
        {
            return ToImageSource(Icon.ExtractAssociatedIcon(path));
        }

        private static bool IsNotModernApp(string path)
        {
            return !path.Contains(@"Program Files\WindowsApps");
        }

        [DllImport("gdi32.dll", SetLastError = true)]
        private static extern bool DeleteObject(IntPtr hObject);

        private static ImageSource ToImageSource(Icon icon)
        {
            var bitmap = icon.ToBitmap();
            var hBitmap = bitmap.GetHbitmap();

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
}