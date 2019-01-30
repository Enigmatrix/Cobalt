using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reactive.Disposables;
using System.Text;
using System.Threading.Tasks;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Data;
using System.Windows.Documents;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Imaging;
using System.Windows.Navigation;
using System.Windows.Shapes;
using ReactiveUI;

namespace Cobalt
{
    /// <summary>
    /// Interaction logic for AppUsageView.xaml
    /// </summary>
    public partial class AppUsageView
    {
        public AppUsageView()
        {
            InitializeComponent();
            this.WhenActivated(regs =>
            {
                this.OneWayBind(ViewModel,
                    vm => vm.Path,
                    v => v.Path.Text)
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                    vm => vm.Image,
                    v => v.Image.Source,
                    x => x == null ? null : LoadImage(x.Value))
                    .DisposeWith(regs);
            });
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
