using Cobalt.Common.UI.ViewModels;
using System;
using System.Collections.Generic;
using System.Linq;
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

namespace Cobalt.Views.Dialogs
{
    /// <summary>
    /// Interaction logic for EditAppAlertDialog.xaml
    /// </summary>
    public partial class EditAppAlertDialog
    {
        public EditAppAlertDialog()
        {
            InitializeComponent();
        }

        public override void Prepare(object[] args)
        {
            var vm = (AppAlertViewModel)args[0];
            DataContext = vm;
        }
    }
}
