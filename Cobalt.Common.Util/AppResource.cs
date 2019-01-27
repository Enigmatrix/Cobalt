using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Linq;
using System.Xml.Linq;

namespace Cobalt.Common.Util
{

    public class AppResource
    {
        private static readonly string[] _colors =
        {
            //from https://flatuicolors.com/palette/es
            Rgb(64, 64, 122),
            Rgb(112, 111, 211),
            Rgb(247, 241, 227),
            Rgb(52, 172, 224),
            Rgb(51, 217, 178),

            Rgb(44, 44, 84),
            Rgb(71, 71, 135),
            Rgb(170, 166, 157),
            Rgb(34, 112, 147),
            Rgb(33, 140, 116),

            Rgb(255, 82, 82),
            Rgb(255, 121, 63),
            Rgb(209, 204, 192),
            Rgb(255, 177, 66),
            Rgb(255, 218, 121),

            Rgb(179, 57, 57),
            Rgb(205, 97, 51),
            Rgb(132, 129, 122),
            Rgb(204, 142, 53),
            Rgb(204, 174, 98),

            //from http://ethanschoonover.com/solarized
            Rgb(65, 181, 137),
            Rgb(203, 75, 22),
            Rgb(220, 50, 47),
            Rgb(211, 54, 130),
            Rgb(108, 113, 196),

            Rgb(38, 139, 210),

            Rgb(42, 161, 152),

            Rgb(133, 153, 0)
        };

        public static string GetAppName(string appPath)
        {
            try
            {
                return FileVersionInfo.GetVersionInfo(appPath).FileDescription;
            }
            catch (FileNotFoundException)
            {
                return null;
            }
        }

        public static (byte[], string) GetAppIconAndColor(string appPath)
        {
            var bytes = Get(appPath);
            var rng = new Random();
            var col = _colors[rng.Next(_colors.Length)];
            return (bytes, col);
        }


        private static string Rgb(byte r, byte g, byte b)
        {
            return "#" + r.ToString("x2") + g.ToString("x2") + b.ToString("x2");
        }

        private static byte[] Get(string path)
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


        private static bool IsNotModernApp(string path)
        {
            return !path.Contains(@"Program Files\WindowsApps");
        }


        private static byte[] GetModernAppIcon(string path)
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

            return imagePath != null ? File.ReadAllBytes(imagePath) : null;
        }

        private static byte[] GetNormalAppIcon(string path)
        {
            using (var ms = new MemoryStream())
            {
                Icon.ExtractAssociatedIcon(path)?.ToBitmap().Save(ms, ImageFormat.Png);
                return ms.ToArray();
            }
        }
    }}
