using System;
using System.Windows;
using System.Windows.Controls;

namespace Cobalt.Views.Dialogs
{
    public class Dialog : UserControl {

        public object DialogResult { get; set; }

        public virtual void Prepare(object[] args)
        {

        }
    }
}
