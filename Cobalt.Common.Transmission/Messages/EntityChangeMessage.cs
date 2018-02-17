using Cobalt.Common.Data;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Transmission.Messages
{
    public enum ChangeType
    {
        Add, Remove, Modify
    }

    public class EntityChange<T> where T : Entity
    {
        public EntityChange(T assocEntity, ChangeType change)
        {
            AssociatedEntity = assocEntity;
            ChangeType = change;
        }
        public T AssociatedEntity { get; }
        public ChangeType ChangeType { get; }
    }

    public class EntityChangeMessage<T> : MessageBase where T : Entity
    {
        public EntityChangeMessage(EntityChange<T> change)
        {
            Change = change;
        }
        public EntityChange<T> Change { get; set; }
    }
}
