using System;
using System.IO;
using System.Reactive.Disposables;
using System.Windows;
using System.Windows.Media.Imaging;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using ReactiveUI;

namespace Cobalt.Common.Views.Entities
{
    /// <summary>
    ///     Interaction logic for AppView.xaml
    /// </summary>
    public partial class AppView
    {
        public AppView()
        {
            InitializeComponent();

            this.WhenActivated(regs =>
            {
                this.OneWayBind(ViewModel,
                        vm => vm.Name,
                        v => v.AppName.Text)
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                        vm => vm.Description,
                        v => v.Description.Text)
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                        vm => vm.Icon,
                        v => v.Icon.Source,
                        bytes =>
                        {
                            if (bytes == null) return null; // TODO placeholder loading
                            using var mem = new MemoryStream(bytes) {Position = 0};
                            var img = new BitmapImage();
                            img.BeginInit();
                            img.CacheOption = BitmapCacheOption.OnLoad;
                            img.StreamSource = mem;
                            img.EndInit();
                            img.Freeze();
                            return img;
                        })
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                        vm => vm.Identity,
                        v => v.IdentityType0.Visibility,
                        ident => ident.IsWin32 ? Visibility.Visible : Visibility.Collapsed)
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                        vm => vm.Identity,
                        v => v.IdentityType1.Visibility,
                        ident => ident.IsUWP ? Visibility.Visible : Visibility.Collapsed)
                    .DisposeWith(regs);

                this.OneWayBind(ViewModel,
                        vm => vm.Identity,
                        v => v.IdentityText1.Text,
                        ident => ident switch
                        {
                            AppIdentity.UWP uwp => uwp.AUMID,
                            AppIdentity.Win32 win32 => win32.Path,
                            _ => throw new ArgumentOutOfRangeException(nameof(ident))
                        })
                    .DisposeWith(regs);
            });
        }
    }
}