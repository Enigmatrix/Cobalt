using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Controls
{
    public class AppLayout : ContentControl
    {
        static AppLayout()
        {
            DefaultStyleKeyProperty.OverrideMetadata(typeof(AppLayout), new FrameworkPropertyMetadata(typeof(AppLayout)));
        }


        public override void OnApplyTemplate()
        {
            base.OnApplyTemplate();
            VisualStateManager.GoToState(this, "Normal", true);
            /*
            var contentCover = (FrameworkElement)GetTemplateChild("PART_ContentCover");
            contentCover.MouseDown += (e, a) =>
            {
                IsMenuOpen = false;
            };*/
        }

        private static void FrameworkPropertyChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            /*
            var nav = (AppLayout)d;
            if (nav.IsMenuOpen && nav.IsSlideOverMenu)
            {
                VisualStateManager.GoToState(nav, "SlideDrawerOpen", true);
            }
            else if (nav.IsMenuOpen)
            {
                VisualStateManager.GoToState(nav, "DrawerOpen", true);
            }
            else
            {
                VisualStateManager.GoToState(nav, "Normal", true);
            }*/
        }

        #region Framework Properties
        
        public AppMenuItemCollection MenuItems
        {
            get { return (AppMenuItemCollection)GetValue(MenuItemsProperty); }
            set { SetValue(MenuItemsProperty, value); }
        }

        public static readonly DependencyProperty MenuItemsProperty =
            DependencyProperty.Register("MenuItems", typeof(AppMenuItemCollection), typeof(AppLayout), new PropertyMetadata(new AppMenuItemCollection()));

        public AppMenuItemCollection OptionItems
        {
            get { return (AppMenuItemCollection)GetValue(OptionItemsProperty); }
            set { SetValue(OptionItemsProperty, value); }
        }

        public static readonly DependencyProperty OptionItemsProperty =
            DependencyProperty.Register("OptionItems", typeof(AppMenuItemCollection), typeof(AppLayout), new PropertyMetadata(new AppMenuItemCollection()));

        public bool IsSlideOverMenu
        {
            get { return (bool)GetValue(IsSlideOverMenuProperty); }
            set { SetValue(IsSlideOverMenuProperty, value); }
        }

        public static readonly DependencyProperty IsSlideOverMenuProperty =
            DependencyProperty.Register("IsSlideOverMenu", typeof(bool), typeof(AppLayout),
                new PropertyMetadata(false, FrameworkPropertyChanged));

        public bool IsMenuOpen
        {
            get { return (bool)GetValue(IsMenuOpenProperty); }
            set { SetValue(IsMenuOpenProperty, value); }
        }

        public static readonly DependencyProperty IsMenuOpenProperty =
            DependencyProperty.Register("IsMenuOpen", typeof(bool), typeof(AppLayout), new PropertyMetadata(false, FrameworkPropertyChanged));

        #endregion
    }
}
