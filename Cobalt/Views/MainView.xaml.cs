using System;
using System.Windows;
using System.Windows.Input;
using Cobalt.Common.IoC;

namespace Cobalt.Views
{
    public partial class MainView
    {
        public MainView()
        {
            InitializeComponent();
        }

        private void MainView_OnClosed(object sender, EventArgs e)
        {
            Application.Current.Shutdown();
        }
    }
}