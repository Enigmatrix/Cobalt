using MaterialDesignThemes.Wpf;
using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Controls
{
    public class AppMenuItem : Control
    {
        static AppMenuItem()
        {
            DefaultStyleKeyProperty.OverrideMetadata(typeof(AppMenuItem), new FrameworkPropertyMetadata(typeof(AppMenuItem)));
        }

        #region Properties

        public PackIconKind Icon
        {
            get { return (PackIconKind)GetValue(IconProperty); }
            set { SetValue(IconProperty, value); }
        }

        public static readonly DependencyProperty IconProperty =
            DependencyProperty.Register("Icon", typeof(PackIconKind), typeof(AppMenuItem), new PropertyMetadata(PackIconKind.AccessPoint));

        public string Description
        {
            get { return (string)GetValue(DescriptionProperty); }
            set { SetValue(DescriptionProperty, value); }
        }

        public static readonly DependencyProperty DescriptionProperty =
            DependencyProperty.Register("Description", typeof(string), typeof(AppMenuItem), new PropertyMetadata(""));

        //TODO some sort of command/link to fire when menu item clicked


        #endregion
    }
}
