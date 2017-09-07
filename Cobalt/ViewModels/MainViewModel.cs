using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.UI;

namespace Cobalt.ViewModels
{
    public interface IMainViewModel : IViewModel
    {
        
    }
    public class MainViewModel : ViewModelBase, IMainViewModel
    {
        private string _booty;

        public MainViewModel()
        {
            Booty = "SMELLS NICE";
        }

        public string Booty
        {
            get => _booty;
            set => Set(ref _booty, value);
        }
    }
}
