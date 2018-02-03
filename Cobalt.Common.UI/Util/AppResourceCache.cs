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
        private readonly Dictionary<string, SolidColorBrush> _colorMapper = new Dictionary<string, SolidColorBrush>();

        private readonly SolidColorBrush[] _colors =
        {
            //from https://flatuicolors.com/palette/es
            rgb(64, 64, 122),
            rgb(112, 111, 211),
            rgb(247, 241, 227),
            rgb(52, 172, 224),
            rgb(51, 217, 178),

            rgb(44, 44, 84),
            rgb(71, 71, 135),
            rgb(170, 166, 157),
            rgb(34, 112, 147),
            rgb(33, 140, 116),

            rgb(255, 82, 82),
            rgb(255, 121, 63),
            rgb(209, 204, 192),
            rgb(255, 177, 66),
            rgb(255, 218, 121),

            rgb(179, 57, 57),
            rgb(205, 97, 51),
            rgb(132, 129, 122),
            rgb(204, 142, 53),
            rgb(204, 174, 98),

            //from http://ethanschoonover.com/solarized
            rgb(65, 181, 137),
            rgb(203, 75, 22),
            rgb(220, 50, 47),
            rgb(211, 54, 130),
            rgb(108, 113, 196),

            rgb(38, 139, 210),

            rgb(42, 161, 152),

            rgb(133, 153, 0)
        };

        private readonly Dictionary<string, ImageSource> _iconMapper = new Dictionary<string, ImageSource>();
        private readonly Dictionary<string, string> _nameMapper = new Dictionary<string, string>();

        private readonly Random _rng = new Random();
        public static AppResourceCache Instance { get; } = new AppResourceCache();

        private static SolidColorBrush rgb(byte r, byte g, byte b)
        {
            return new SolidColorBrush(Color.FromRgb(r, g, b));
        }


        private SolidColorBrush RandomColor()
        {
            return _colors[_rng.Next(_colors.Length)];
        }

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