using System;
using System.Windows;

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