using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{

    public class EntityViewModel : ViewModelBase
    {
        private int _id;

        public EntityViewModel(Entity entity)
        {
            
        }

        public int Id
        {
            get => _id;
            set => Set(ref _id, value);
        }
    }
}
