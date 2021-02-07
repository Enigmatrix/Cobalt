using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Tray
{
    /// <summary>
    ///     Follow steps 1a or 1b and then 2 to use this custom control in a XAML file.
    ///     Step 1a) Using this custom control in a XAML file that exists in the current project.
    ///     Add this XmlNamespace attribute to the root element of the markup file where it is
    ///     to be used:
    ///     xmlns:MyNamespace="clr-namespace:Cobalt.Tray"
    ///     Step 1b) Using this custom control in a XAML file that exists in a different project.
    ///     Add this XmlNamespace attribute to the root element of the markup file where it is
    ///     to be used:
    ///     xmlns:MyNamespace="clr-namespace:Cobalt.Tray;assembly=Cobalt.Tray"
    ///     You will also need to add a project reference from the project where the XAML file lives
    ///     to this project and Rebuild to avoid compilation errors:
    ///     Right click on the target project in the Solution Explorer and
    ///     "Add Reference"->"Projects"->[Browse to and select this project]
    ///     Step 2)
    ///     Go ahead and use your control in the XAML file.
    ///     <MyNamespace:CustomGrid />
    /// </summary>
    public class CustomGrid : Grid
    {
        static CustomGrid()
        {
            DefaultStyleKeyProperty.OverrideMetadata(typeof(CustomGrid),
                new FrameworkPropertyMetadata(typeof(CustomGrid)));
            MarginProperty.OverrideMetadata(typeof(CustomGrid), new FrameworkPropertyMetadata(MarginChg));
        }

        // Stupid TabControl sets the margin of its Parent, we set it back to override it. :)
        private static void MarginChg(DependencyObject d, DependencyPropertyChangedEventArgs e)
        {
            var self = (CustomGrid) d;
            self.Margin = new Thickness(0);
        }
    }
}