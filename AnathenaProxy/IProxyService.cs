﻿namespace AnathenaProxy
{
    using System.ServiceModel;

    [ServiceContract()]
    public interface IProxyService : IProxyAssembler, IProxyClr
    {
    }
    //// End interface
}
//// End namespace