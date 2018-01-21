using System.Windows;
using System.Windows.Controls;
using MaterialDesignThemes.Wpf;

namespace Cobalt.Views.Controls
{
    public class AppMenuItem : Control
    {
        static AppMenuItem()
        {
            DefaultStyleKeyProperty.OverrideMetadata(typeof(AppMenuItem),
                new FrameworkPropertyMetadata(typeof(AppMenuItem)));
        }

        #region Properties

        public PackIconKind Icon
        {
            get => (PackIconKind) GetValue(IconProperty);
            set => SetValue(IconProperty, value);
        }

        public static readonly DependencyProperty IconProperty =
            DependencyProperty.Register("Icon", typeof(PackIconKind), typeof(AppMenuItem),
                new PropertyMetadata(PackIconKind.AccessPoint));

        public string Description
        {
            get => (string) GetValue(DescriptionProperty);
            set => SetValue(DescriptionProperty, value);
        }

        public static readonly DependencyProperty DescriptionProperty =
            DependencyProperty.Register("Description", typeof(string), typeof(AppMenuItem), new PropertyMetadata(""));

        //TODO some sort of command/link to fire when menu item clicked

        #endregion
    }
}