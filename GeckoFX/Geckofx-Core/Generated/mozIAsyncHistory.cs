// --------------------------------------------------------------------------------------------
// Version: MPL 1.1/GPL 2.0/LGPL 2.1
// 
// The contents of this file are subject to the Mozilla Public License Version
// 1.1 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
// http://www.mozilla.org/MPL/
// 
// Software distributed under the License is distributed on an "AS IS" basis,
// WITHOUT WARRANTY OF ANY KIND, either express or implied. See the License
// for the specific language governing rights and limitations under the
// License.
// 
// <remarks>
// Generated by IDLImporter from file mozIAsyncHistory.idl
// 
// You should use these interfaces when you access the COM objects defined in the mentioned
// IDL/IDH file.
// </remarks>
// --------------------------------------------------------------------------------------------
namespace Gecko
{
    using System;
    using System.Runtime.CompilerServices;
    using System.Runtime.InteropServices;


    /// <summary>
    ///This Source Code Form is subject to the terms of the Mozilla Public
    /// License, v. 2.0. If a copy of the MPL was not distributed with this
    /// file, You can obtain one at http://mozilla.org/MPL/2.0/. </summary>
    [ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("41e4ccc9-f0c8-4cd7-9753-7a38514b8488")]
	public interface mozIVisitInfo
	{
		
		/// <summary>
        /// The machine-local (internal) id of the visit.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		long GetVisitIdAttribute();
		
		/// <summary>
        /// The time the visit occurred.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		long GetVisitDateAttribute();
		
		/// <summary>
        /// The transition type used to get to this visit.  One of the TRANSITION_TYPE
        /// constants on nsINavHistory.
        ///
        /// @see nsINavHistory.idl
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		uint GetTransitionTypeAttribute();
		
		/// <summary>
        /// The referring URI of this visit.  This may be null.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIURI GetReferrerURIAttribute();
	}
	
	/// <summary>mozIPlaceInfo </summary>
	[ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("ad83e137-c92a-4b7b-b67e-0a318811f91e")]
	public interface mozIPlaceInfo
	{
		
		/// <summary>
        /// The machine-local (internal) id of the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		long GetPlaceIdAttribute();
		
		/// <summary>
        /// The globally unique id of the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetGuidAttribute([MarshalAs(UnmanagedType.LPStruct)] nsACStringBase aGuid);
		
		/// <summary>
        /// The URI of the place.
        /// </summary>
		[return: MarshalAs(UnmanagedType.Interface)]
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		nsIURI GetUriAttribute();
		
		/// <summary>
        /// The title associated with the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetTitleAttribute([MarshalAs(UnmanagedType.CustomMarshaler, MarshalType = "Gecko.CustomMarshalers.AStringMarshaler")] nsAStringBase aTitle);
		
		/// <summary>
        /// The frecency of the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		long GetFrecencyAttribute();
		
		/// <summary>
        /// An array of mozIVisitInfo objects for the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		Gecko.JsVal GetVisitsAttribute(System.IntPtr jsContext);
	}
	
	/// <summary>
    /// Shared Callback interface for mozIAsyncHistory methods. The semantics
    /// for each method are detailed in mozIAsyncHistory.
    /// </summary>
	[ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("1f266877-2859-418b-a11b-ec3ae4f4f93d")]
	public interface mozIVisitInfoCallback
	{
		
		/// <summary>
        /// Called when the given place could not be processed.
        ///
        /// @param aResultCode
        /// nsresult indicating the failure reason.
        /// @param aPlaceInfo
        /// The information that was given to the caller for the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void HandleError(int aResultCode, mozIPlaceInfo aPlaceInfo);
		
		/// <summary>
        /// Called for each place processed successfully.
        ///
        /// @param aPlaceInfo
        /// The current info stored for the place.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void HandleResult(mozIPlaceInfo aPlaceInfo);
		
		/// <summary>
        /// Called when all records were processed.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void HandleCompletion();
	}
	
	/// <summary>mozIVisitedStatusCallback </summary>
	[ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("994092bf-936f-449b-8dd6-0941e024360d")]
	public interface mozIVisitedStatusCallback
	{
		
		/// <summary>
        /// Notifies whether a certain URI has been visited.
        ///
        /// @param aURI
        /// URI being notified about.
        /// @param aVisitedStatus
        /// The visited status of aURI.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void IsVisited([MarshalAs(UnmanagedType.Interface)] nsIURI aURI, [MarshalAs(UnmanagedType.U1)] bool aVisitedStatus);
	}
	
	/// <summary>mozIAsyncHistory </summary>
	[ComImport()]
	[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
	[Guid("1643EFD2-A329-4733-A39D-17069C8D3B2D")]
	public interface mozIAsyncHistory
	{
		
		/// <summary>
        /// Gets the available information for the given array of places, each
        /// identified by either nsIURI or places GUID (string).
        ///
        /// The retrieved places info objects DO NOT include the visits data (the
        /// |visits| attribute is set to null).
        ///
        /// If a given place does not exist in the database, aCallback.handleError is
        /// called for it with NS_ERROR_NOT_AVAILABLE result code.
        ///
        /// @param aPlaceIdentifiers
        /// The place[s] for which to retrieve information, identified by either
        /// a single place GUID, a single URI, or a JS array of URIs and/or GUIDs.
        /// @param aCallback
        /// A mozIVisitInfoCallback object which consists of callbacks to be
        /// notified for successful or failed retrievals.
        /// If there's no information available for a given place, aCallback
        /// is called with a stub place info object, containing just the provided
        /// data (GUID or URI).
        ///
        /// @throws NS_ERROR_INVALID_ARG
        /// - Passing in NULL for aPlaceIdentifiers or aCallback.
        /// - Not providing at least one valid GUID or URI.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void GetPlacesInfo(ref Gecko.JsVal aPlaceIdentifiers, mozIVisitInfoCallback aCallback, System.IntPtr jsContext);
		
		/// <summary>
        /// Adds a set of visits for one or more mozIPlaceInfo objects, and updates
        /// each mozIPlaceInfo's title or guid.
        ///
        /// aCallback.handleResult is called for each visit added.
        ///
        /// @param aPlaceInfo
        /// The mozIPlaceInfo object[s] containing the information to store or
        /// update.  This can be a single object, or an array of objects.
        /// @param [optional] aCallback
        /// A mozIVisitInfoCallback object which consists of callbacks to be
        /// notified for successful and/or failed changes.
        ///
        /// @throws NS_ERROR_INVALID_ARG
        /// - Passing in NULL for aPlaceInfo.
        /// - Not providing at least one valid guid, or uri for all
        /// mozIPlaceInfo object[s].
        /// - Not providing an array or nothing for the visits property of
        /// mozIPlaceInfo.
        /// - Not providing a visitDate and transitionType for each
        /// mozIVisitInfo.
        /// - Providing an invalid transitionType for a mozIVisitInfo.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void UpdatePlaces(ref Gecko.JsVal aPlaceInfo, mozIVisitInfoCallback aCallback, System.IntPtr jsContext);
		
		/// <summary>
        /// Checks if a given URI has been visited.
        ///
        /// @param aURI
        /// The URI to check for.
        /// @param aCallback
        /// A mozIVisitStatusCallback object which receives the visited status.
        /// </summary>
		[MethodImpl(MethodImplOptions.InternalCall, MethodCodeType=MethodCodeType.Runtime)]
		void IsURIVisited([MarshalAs(UnmanagedType.Interface)] nsIURI aURI, mozIVisitedStatusCallback aCallback);
	}
}
