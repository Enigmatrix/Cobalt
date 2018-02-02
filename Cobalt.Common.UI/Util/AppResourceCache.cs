using System;
using System.Collections.Generic;
using System.ComponentModel;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Xml.Linq;
using Color = System.Windows.Media.Color;

namespace Cobalt.Common.UI.Util
{
    public class AppResourceCache
    {
        public static AppResourceCache Instance { get; } = new AppResourceCache();

        private readonly Dictionary<string, ImageSource> _iconMapper = new Dictionary<string, ImageSource>();
        private readonly Dictionary<string, string> _nameMapper = new Dictionary<string, string>();
        private readonly Dictionary<string, SolidColorBrush> _colorMapper = new Dictionary<string, SolidColorBrush>();

        private readonly Random _rng = new Random();
        private static SolidColorBrush rgb(byte r, byte g, byte b)
        {
            return new SolidColorBrush(Color.FromRgb(r,g,b));
        }

        private readonly SolidColorBrush[] _colors = {
            rgb(252, 92, 101),
            rgb(235, 59, 90),
            rgb(254, 211, 48),
            rgb(75, 123, 236),

            rgb(253, 150, 68),
            rgb(250, 130, 49),
            rgb(247, 183, 49),
            rgb(38, 222, 129),

            rgb(32, 191, 107),
            rgb(43, 203, 186),
            rgb(15, 185, 177),
            rgb(69, 170, 242),

            rgb(165, 94, 234),
            rgb(209, 216, 224),
            rgb(119, 140, 163),
            rgb(45, 152, 218),

            rgb(56, 103, 214),
            rgb(136, 84, 208),
            rgb(165, 177, 194),
            rgb(75, 101, 132)
        };

        private SolidColorBrush RandomColor() => _colors[_rng.Next(_colors.Length)];

        public SolidColorBrush GetColor(string path)
        {
            if (!_colorMapper.ContainsKey(path))
                _colorMapper[path] = RandomColor();
            return _colorMapper[path];
        }

        public string GetName(string path)
        {
            if (!_nameMapper.ContainsKey(path))
                try
                {
                    _nameMapper[path] = FileVersionInfo.GetVersionInfo(path).FileDescription;
                }
                catch (FileNotFoundException)
                {
                    //todo i18nilize this
                    _nameMapper[path] = "Not enough access";
                }

            return _nameMapper[path];
        }

        public ImageSource GetIcon(string pathStr)
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

            if (pathStr == null) return null;
            if (!_iconMapper.ContainsKey(pathStr)) _iconMapper[pathStr] = Get(pathStr);
            return _iconMapper[pathStr];
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
            return File.Exists(imagePath) ? new BitmapImage(new Uri($@"file:/{imagePath}")) : null;
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
