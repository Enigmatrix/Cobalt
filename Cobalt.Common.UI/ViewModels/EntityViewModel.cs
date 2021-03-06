﻿using System;
using System.Reactive.Linq;
using Cobalt.Common.Analysis;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.ViewModels
{
    public class EntityViewModel<T> : EntityViewModel
        where T : Entity
    {
        public new T Entity { get; set; }

        public EntityViewModel(T entity) : base(entity)
        {
            Entity = entity;
        }
    }

    public class EntityViewModel : ViewModelBase
    {
        public EntityViewModel(Entity entity)
        {
            Entity = entity;
        }

        public int Id { get; set; }
        
        private IResourceScope _resources;
        private IDbRepository _repository;
        private IAppStatsStreamService _appStats;

        public IResourceScope Resources => _resources ?? (_resources = IoCService.Instance.Resolve<IResourceScope>().Subscope());

        protected IDbRepository Repository => _repository ?? (_repository = Resources.Resolve<IDbRepository>());
        
        protected IAppStatsStreamService AppStats => _appStats ?? (_appStats = Resources.Resolve<IAppStatsStreamService>());

        public Entity Entity { get; }

        public override void Dispose()
        {
            base.Dispose();
            Resources.Dispose();
        }
    }
}