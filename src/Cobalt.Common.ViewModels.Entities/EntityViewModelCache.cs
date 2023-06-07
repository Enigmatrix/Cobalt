using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.ViewModels.Entities
{
    public interface IEntityViewModelCache
    {
        AlertViewModel Alert(Alert tag);
        AppViewModel App(App tag);
        TagViewModel Tag(Tag tag);
    }
    public class EntityViewModelCache
    {
    }
}
